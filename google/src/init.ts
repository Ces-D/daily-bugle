import { googleCalenderClient } from "./client.js";
import { createBaseCalendar } from "./calendar.js";

const COMMANDS = {
  test: "test",
  createBaseCalendar: "create_base_calendar",
};

const main = async () => {
  const [, , command, ...args] = process.argv;
  if (Object.values(COMMANDS).includes(command)) {
    switch (command) {
      case COMMANDS.createBaseCalendar: {
        const [credentialFilePath] = args;
        const calendarClient = await googleCalenderClient(credentialFilePath);
        return createBaseCalendar(calendarClient);
      }
      case COMMANDS.test: {
        const wait: Promise<String> = new Promise((resolve) => {
          setTimeout(() => {
            resolve("Success");
          }, 2_000);
        });
        return wait;
      }
      default: {
        throw new Error("Not supported case");
      }
    }
  } else {
    throw new Error(`Not supported command: ${command}`);
  }
};

main().then((res) => {
  console.log(res);
});
