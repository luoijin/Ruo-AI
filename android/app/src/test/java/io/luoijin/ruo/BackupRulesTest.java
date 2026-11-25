package io.luoijin.ruo;

import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.HashSet;
import java.util.Set;
import javax.xml.parsers.DocumentBuilderFactory;
import org.junit.Test;
import org.w3c.dom.Element;
import org.w3c.dom.NodeList;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertTrue;

public class BackupRulesTest {
    @Test
    public void legacyBackupRulesExcludeDatabaseAndKeyMaterial() throws Exception {
        Element root = parseRules("backup_rules.xml");
        assertEquals("full-backup-content", root.getTagName());
        assertExcluded(root, "database", "sharedpref", "file");
    }

    @Test
    public void cloudAndDeviceTransferRulesExcludeDatabaseAndKeyMaterial() throws Exception {
        Element root = parseRules("data_extraction_rules.xml");
        Element cloudBackup = child(root, "cloud-backup");
        Element deviceTransfer = child(root, "device-transfer");

        assertExcluded(cloudBackup, "database", "sharedpref", "file");
        assertExcluded(deviceTransfer, "database", "sharedpref", "file");
    }

    private static Element parseRules(String name) throws Exception {
        Path current = Paths.get(System.getProperty("user.dir")).toAbsolutePath();
        while (current != null) {
            Path direct = current.resolve("src/main/res/xml").resolve(name);
            Path nested = current.resolve("android/app/src/main/res/xml").resolve(name);
            Path candidate = Files.exists(direct) ? direct : nested;
            if (Files.exists(candidate)) {
                return DocumentBuilderFactory.newInstance()
                        .newDocumentBuilder()
                        .parse(candidate.toFile())
                        .getDocumentElement();
            }
            current = current.getParent();
        }
        throw new AssertionError("Backup rules file not found: " + name);
    }

    private static Element child(Element parent, String tagName) {
        NodeList children = parent.getElementsByTagName(tagName);
        assertNotNull(children);
        assertTrue("Missing backup section: " + tagName, children.getLength() > 0);
        return (Element) children.item(0);
    }

    private static void assertExcluded(Element parent, String... domains) {
        NodeList excludes = parent.getElementsByTagName("exclude");
        Set<String> rules = new HashSet<>();
        for (int i = 0; i < excludes.getLength(); i++) {
            Element exclude = (Element) excludes.item(i);
            rules.add(exclude.getAttribute("domain") + ":" + exclude.getAttribute("path"));
        }
        for (String domain : domains) {
            assertTrue("Missing root exclusion for " + domain, rules.contains(domain + ":."));
        }
    }
}
