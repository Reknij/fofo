import type { UserNotification, GetUserNotificationsQuery, SetUserNotificationReadedQuery } from "~/models/notification";
import type { GetDatasExtended } from "~/models/util";
import { isLogined } from "~/states/auth";
import { useApiFetch } from "./customFetch";

export function getUserNotifications(query: GetUserNotificationsQuery) {
    return useApiFetch<GetDatasExtended<UserNotification>>(`/user_notifications`, {
        query,
    })
}

export function setUserNotificationReaded(query: SetUserNotificationReadedQuery) {
    return useApiFetch<UserNotification>(`/set_user_notification_readed`, {
        query,
    })
}