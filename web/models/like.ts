
export interface GetLikeStatusQuery {
    flag_ref_id: number;
    flag: LikeStatusFlag;
    created_by_id: number;
}

export interface LikeStatus {
    flag: LikeStatusFlag,
    flag_ref_id: number,
    created_by_id: number,
    created_at: number,
    is_like: boolean,
}

export enum LikeAction {
    Like,
    Dislike,
    Unknown,
}

export interface LikeActionBody {
    flag: LikeStatusFlag,
    action: LikeAction,
}

export enum LikeStatusFlag {
    TargetPost,
    TargetComment,
}