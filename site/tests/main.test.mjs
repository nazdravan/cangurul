// Testele butonului de descărcare: node --test site/tests/
import { test } from "node:test";
import assert from "node:assert/strict";
import { pickDownload, withHash } from "../assets/main.js";

test("windows", () => assert.equal(pickDownload("Mozilla/5.0 (Windows NT 10.0; Win64; x64)", "Win32"), "windows"));
test("mac", () => assert.equal(pickDownload("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)", "MacIntel"), "mac"));
test("linux", () => assert.equal(pickDownload("Mozilla/5.0 (X11; Linux x86_64)", "Linux x86_64"), "linux"));
test("android nu primește AppImage", () => assert.equal(pickDownload("Mozilla/5.0 (Linux; Android 14; Pixel 8)", "Linux armv8l"), null));
test("iPhone", () => assert.equal(pickDownload("Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)", "iPhone"), null));
test("iPad cu UA de Mac", () => assert.equal(pickDownload("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)", "MacIntel", 5), null));
test("Windows 7 primește arhiva pentru Windows 7", () => assert.equal(pickDownload("Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:115.0) Gecko/20100101 Firefox/115.0", "Win32"), "win7"));
test("Windows 8.1 tot arhiva pentru Windows 7", () => assert.equal(pickDownload("Mozilla/5.0 (Windows NT 6.3; Win64; x64)", "Win32"), "win7"));
test("limba păstrează secțiunea", () => assert.equal(withHash("../en/", "#descarca"), "../en/#descarca"));
test("limba fără secțiune", () => assert.equal(withHash("./", ""), "./"));
test("Chromebook", () => assert.equal(pickDownload("Mozilla/5.0 (X11; CrOS x86_64 14541.0.0)", "Linux x86_64"), null));
