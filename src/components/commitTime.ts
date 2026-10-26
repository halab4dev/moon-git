// How the History list shows a commit's time: "Today, 14:32" and
// "Yesterday, 09:10" for the last two days, the full date otherwise. Days
// are calendar days in the local time zone, not "less than 24 hours ago".

const timeFormat = new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" });
const dateTimeFormat = new Intl.DateTimeFormat(undefined, {
  year: "numeric",
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
});

/** Local midnight at the start of `date`'s day. */
function startOfDay(date: Date): number {
  return new Date(date.getFullYear(), date.getMonth(), date.getDate()).getTime();
}

/** `seconds`: since 1970 (as git gives it). `now`: passed in so tests can fix it. */
export function formatCommitTime(seconds: number, now: Date = new Date()): string {
  const date = new Date(seconds * 1000);
  const today = startOfDay(now);
  // Built from the calendar date (not "minus 24 hours"), so a day with a
  // daylight saving change is still exactly one day.
  const yesterday = startOfDay(new Date(now.getFullYear(), now.getMonth(), now.getDate() - 1));
  const day = startOfDay(date);
  if (day === today) {
    return `Today, ${timeFormat.format(date)}`;
  }
  if (day === yesterday) {
    return `Yesterday, ${timeFormat.format(date)}`;
  }
  return dateTimeFormat.format(date);
}
