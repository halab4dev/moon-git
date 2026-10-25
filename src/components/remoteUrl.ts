// How a remote URL is shown in the branch list. A password or token written
// into the URL ("https://me:ghp_xxx@github.com/…") is hidden: the list is on
// screen all the time (CLAUDE.md: never show or log secrets).

/** "https://me:secret@host/x.git" → "https://me:***@host/x.git"; other URLs unchanged. */
export function displayUrl(url: string): string {
  return url.replace(/^([a-z][a-z0-9+.-]*:\/\/[^/@:]*):[^/@]*@/i, "$1:***@");
}
