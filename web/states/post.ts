import type { SafeUserInfo, UserTag } from "~/models/user";
export interface UserAndTag {
    user: SafeUserInfo,
    tag: UserTag,
}
export const usePostUserAndTag = () => useState<UserAndTag|null>('postUser', ()=>null)
