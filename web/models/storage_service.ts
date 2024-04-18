export enum SignedFlag {
    UserAvatar,
    PostCover,
    CategoryCover,
}

export interface GetPresignedUrlQuery {
    signed_flag: SignedFlag;
    filename: string;
}

export interface GetPresignedUrlResult {
    object_url: string;
    presigned_url: string;
}