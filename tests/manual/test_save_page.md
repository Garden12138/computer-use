# Manual Webpage Complete save

1. Grant permissions via `computer-use doctor`
2. `computer-use browser-open-profile ComputerUse`
3. Log in once in that profile if the site needs it
4. `computer-use browser-open-url "$URL"`
5. Confirm the page is visible (not captcha / login wall)
6. `computer-use --pacing normal browser-save-page ./out/page.html --scrolls 8`
7. Confirm `./out/page.html` and `./out/page_files/` exist
8. Confirm no Playwright / CDP / debug port was used
