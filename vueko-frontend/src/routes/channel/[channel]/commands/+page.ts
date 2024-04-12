import type { PageLoad } from "./$types";

interface Commands {
  commands: Command[]
}

interface Command {
  id: string;
  channel: string;
  command: string;
  value: string;
}

export const load: PageLoad = async ({ params }): Promise<Commands> => {
  let response = await fetch("https://vueko.ves.dev/api/v1/channel/" + params.channel + "/commands.json");
  return await response.json();
}
