export interface UserNotification {
    id: number;
    ref_id: number;
    target_user_id: number;
    created_by_id: number;
    created_at: number;
    n_type: UserNotificationType;
    readed: boolean;
  }
  
  export enum UserNotificationType {
    Comment,
    ReplyComment,
    LikePost,
    DislikePost,
    LikeComment,
    DislikeComment,
  }

  export interface GetUserNotificationsQuery {
    index: number,
    limit: number,
    extended: boolean,
    only_unread: boolean,
  }

  export interface SetUserNotificationReadedQuery {
    id: number;
    readed: boolean;
  }