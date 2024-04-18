import type { ServerInfo } from "~/models/server";
import { useApiFetch } from "./customFetch";

export function getServerInfo() {
    return useApiFetch<ServerInfo>(`/server_info`);
}