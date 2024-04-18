import type { SafeUserInfo } from "~/models/user";
export const usePostUser = () => useState<SafeUserInfo | null>('postUser', () => null)
