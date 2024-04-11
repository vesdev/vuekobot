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
  let response = await fetch("http://localhost:45861/api/v1/channel/" + params.channel + "/commands");
  return await response.json();
}
