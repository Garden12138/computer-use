---
name: browser-use
description: Operate Google Chrome as a normal desktop app via computer-use primitives. Open a dedicated profile, type URLs in the omnibox, slow-scroll, and save Webpage Complete. Never attach CDP or Playwright.
---

# browser-use

Chrome is a regular desktop app. Drive it with `computer-use`, not a browser driver.

v0.2 browser recipes (`browser-open-profile`, `browser-open-url`, `browser-save-page`) are **macOS-only**. On Windows/Linux use screenshot + click + hotkey (`ctrl+l`, `ctrl+s`) instead.

## Profile

Use a dedicated Chrome profile (human logs in once):

```bash
computer-use browser-open-profile ComputerUse
```

Do not launch Chrome with remote debugging.

## Open a URL

```bash
computer-use browser-open-url "https://example.com"
```

This focuses Chrome, presses `cmd+l`, types the URL, presses Enter.

## Save page

```bash
computer-use --pacing normal browser-save-page ./out/page.html --scrolls 8
```

The runtime jitter-scrolls to load lazy content, then uses Chrome's native Save sheet and selects **网页，全部** / **Webpage, Complete**. Expect `page.html` plus `page_files/`.

## Stop conditions

If the screenshot or saved HTML shows captcha, login, or risk-control copy, stop and ask a human. Do not retry in a loop. Do not bypass verification.
