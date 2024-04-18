import type { ServerInfo } from "~/models/server";

export const useServerInfo = () => useState<ServerInfo | null>('serverInfo', () => null)
