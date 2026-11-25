use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedEvent {
    Meta {
        client_msg_id: String,
        user_message_id: String,
        assistant_message_id: String,
        detected_language: Option<String>,
        study_step: Option<i32>,
        fallback: bool,
        replayed: bool,
    },
    Delta {
        text: String,
    },
    Done {
        assistant_message_id: String,
        replayed: bool,
    },
    Error {
        code: String,
        retryable: bool,
        retry_after_sec: Option<u64>,
    },
    Unknown {
        event: String,
        data: String,
    },
}

#[derive(Default)]
pub struct SseParser {
    buffer: Vec<u8>,
}

impl SseParser {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn feed(&mut self, chunk: &str) -> Vec<ParsedEvent> {
        self.feed_bytes(chunk.as_bytes())
    }

    pub fn feed_bytes(&mut self, chunk: &[u8]) -> Vec<ParsedEvent> {
        self.buffer.extend_from_slice(chunk);
        let mut events = Vec::new();
        while let Some((start, delimiter_len)) = find_delimiter(&self.buffer) {
            let block = self.buffer[..start].to_vec();
            self.buffer.drain(..start + delimiter_len);
            if let Some(event) = self.parse_block(&block) {
                events.push(event);
            }
        }
        events
    }

    fn parse_block(&self, block: &[u8]) -> Option<ParsedEvent> {
        let block = String::from_utf8_lossy(block).replace("\r\n", "\n");
        let mut event_type = "message";
        let mut data_lines = Vec::new();

        for line in block.lines() {
            if line.starts_with(':') {
                continue;
            }
            if let Some(stripped) = line.strip_prefix("event:") {
                event_type = stripped.trim();
            } else if let Some(stripped) = line.strip_prefix("data:") {
                data_lines.push(stripped.trim_start());
            }
        }

        if data_lines.is_empty() {
            return None;
        }
        let data = data_lines.join("\n");

        match event_type {
            "meta" => {
                #[derive(Deserialize)]
                struct MetaPayload {
                    client_msg_id: String,
                    user_message_id: String,
                    assistant_message_id: String,
                    detected_language: Option<String>,
                    study_step: Option<i32>,
                    #[serde(default)]
                    fallback: bool,
                    #[serde(default)]
                    replayed: bool,
                }
                serde_json::from_str::<MetaPayload>(&data)
                    .ok()
                    .map(|payload| ParsedEvent::Meta {
                        client_msg_id: payload.client_msg_id,
                        user_message_id: payload.user_message_id,
                        assistant_message_id: payload.assistant_message_id,
                        detected_language: payload.detected_language,
                        study_step: payload.study_step,
                        fallback: payload.fallback,
                        replayed: payload.replayed,
                    })
                    .or_else(|| {
                        Some(ParsedEvent::Unknown {
                            event: event_type.to_string(),
                            data,
                        })
                    })
            }
            "delta" => {
                #[derive(Deserialize)]
                struct DeltaPayload {
                    text: String,
                }
                serde_json::from_str::<DeltaPayload>(&data)
                    .ok()
                    .map(|payload| ParsedEvent::Delta { text: payload.text })
                    .or_else(|| {
                        Some(ParsedEvent::Unknown {
                            event: event_type.to_string(),
                            data,
                        })
                    })
            }
            "done" => {
                #[derive(Deserialize)]
                struct DonePayload {
                    assistant_message_id: String,
                    #[serde(default)]
                    replayed: bool,
                }
                serde_json::from_str::<DonePayload>(&data)
                    .ok()
                    .map(|payload| ParsedEvent::Done {
                        assistant_message_id: payload.assistant_message_id,
                        replayed: payload.replayed,
                    })
                    .or_else(|| {
                        Some(ParsedEvent::Unknown {
                            event: event_type.to_string(),
                            data,
                        })
                    })
            }
            "error" => {
                #[derive(Deserialize)]
                struct ErrorPayload {
                    code: String,
                    #[serde(default)]
                    retryable: bool,
                    retry_after_sec: Option<u64>,
                }
                serde_json::from_str::<ErrorPayload>(&data)
                    .ok()
                    .map(|payload| ParsedEvent::Error {
                        code: payload.code,
                        retryable: payload.retryable,
                        retry_after_sec: payload.retry_after_sec,
                    })
                    .or_else(|| {
                        Some(ParsedEvent::Unknown {
                            event: event_type.to_string(),
                            data,
                        })
                    })
            }
            _ => Some(ParsedEvent::Unknown {
                event: event_type.to_string(),
                data,
            }),
        }
    }
}

fn find_delimiter(buffer: &[u8]) -> Option<(usize, usize)> {
    for (index, window) in buffer.windows(2).enumerate() {
        if window == b"\n\n" {
            return Some((index, 2));
        }
    }
    for (index, window) in buffer.windows(4).enumerate() {
        if window == b"\r\n\r\n" {
            return Some((index, 4));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_meta_delta_and_done_events_across_chunks() {
        let mut parser = SseParser::new();
        let meta = parser.feed(
            "event: meta\ndata: {\"client_msg_id\":\"client-1\",\"user_message_id\":\"user-1\",\"assistant_message_id\":\"assistant-1\",\"detected_language\":\"tl-PH\",\"fallback\":false}\n\n",
        );
        assert_eq!(
            meta,
            vec![ParsedEvent::Meta {
                client_msg_id: "client-1".to_string(),
                user_message_id: "user-1".to_string(),
                assistant_message_id: "assistant-1".to_string(),
                detected_language: Some("tl-PH".to_string()),
                study_step: None,
                fallback: false,
                replayed: false,
            }]
        );

        assert!(parser
            .feed("event: delta\ndata: {\"text\":\"Kumusta ")
            .is_empty());
        let delta = parser.feed("🌱\"}\n\n");
        assert_eq!(
            delta,
            vec![ParsedEvent::Delta {
                text: "Kumusta 🌱".to_string()
            }]
        );
        assert_eq!(
            parser.feed("event: done\ndata: {\"assistant_message_id\":\"assistant-1\",\"replayed\":true}\n\n"),
            vec![ParsedEvent::Done {
                assistant_message_id: "assistant-1".to_string(),
                replayed: true,
            }]
        );
    }

    #[test]
    fn preserves_multibyte_text_split_inside_a_utf8_codepoint() {
        let mut parser = SseParser::new();
        let mut bytes = b"event: delta\ndata: {\"text\":\"".to_vec();
        bytes.extend_from_slice(&[0xF0, 0x9F]);
        assert!(parser.feed_bytes(&bytes).is_empty());
        assert_eq!(
            parser.feed_bytes(&[0x8C, 0xB1, b'"', b'}', b'\n', b'\n']),
            vec![ParsedEvent::Delta {
                text: "🌱".to_string()
            }]
        );
    }

    #[test]
    fn ignores_server_error_message_and_parses_only_machine_fields() {
        let mut parser = SseParser::new();
        assert_eq!(
            parser.feed("event: error\ndata: {\"code\":\"RATE_LIMIT_EXCEEDED\",\"message\":\"private input echoed\",\"retryable\":true,\"retry_after_sec\":7}\n\n"),
            vec![ParsedEvent::Error {
                code: "RATE_LIMIT_EXCEEDED".to_string(),
                retryable: true,
                retry_after_sec: Some(7),
            }]
        );
    }
}
