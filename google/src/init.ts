import { calendar_v3 } from "googleapis";

/** Constants */
const DAILY_BUGLE_CALENDAR = {
  summary: "Daily-Bugle",
  description:
    "This calendar keeps track of everything happening in the Daily-Bugle project. It includes reminders, start dates, and important events set by the project app. Think of it as the project’s command center — a simple place to stay on top of deadlines, milestones, and day-to-day updates.",
  backgroundColor: "#FFED10",
  foregroundColor: "#E20025",
  selected: true,
} as const;

function hasBaseCalendar(calendarList: calendar_v3.Schema$CalendarList) {
  if (calendarList.items && calendarList.items.length > 0) {
    const hasCalendar = calendarList.items.findIndex(
      (v) =>
        typeof v.summary === "string" &&
        v.summary === DAILY_BUGLE_CALENDAR.summary &&
        !v.primary,
    );
    return hasCalendar > -1;
  } else {
    return false;
  }
}

export async function createBaseCalendar(
  client: calendar_v3.Calendar,
): Promise<calendar_v3.Schema$Calendar> {
  const userCalendars = await client.calendarList.list({});
  if (!hasBaseCalendar(userCalendars.data)) {
    const res = await client.calendars.insert({
      requestBody: {
        summary: DAILY_BUGLE_CALENDAR.summary,
        description: DAILY_BUGLE_CALENDAR.description,
        timeZone: "America/New_York",
      },
    });
    if (res.ok) {
      console.info(
        `Successfully created ${DAILY_BUGLE_CALENDAR.summary} calendar`,
      );
      if (res.data.id) {
        const up = await client.calendarList.update({
          calendarId: res.data.id,
          requestBody: {
            backgroundColor: DAILY_BUGLE_CALENDAR.backgroundColor,
            foregroundColor: DAILY_BUGLE_CALENDAR.foregroundColor,
            selected: DAILY_BUGLE_CALENDAR.selected,
          },
        });
        if (up.ok) {
          console.info(`Successfully themed ${DAILY_BUGLE_CALENDAR} calendar`);
        }
      }
      return res.data;
    } else {
      throw new Error(
        `Failed to create ${DAILY_BUGLE_CALENDAR.summary} calendar`,
        { cause: await res.json() },
      );
    }
  } else {
    throw new Error(`${DAILY_BUGLE_CALENDAR} calendar already exists`);
  }
}

// TODO: create calendar events
