export enum DetailErrorCode {
    InternalError = 10000,
    UnsupportedApi,
    IpAddressNotFound,
    TooManyRequests,

    GetVerificationError = 10100,
    VerificationFailed,
    VerificationNotFound,

    LoginRequired = 10200,
    NoLoginRequired,
    NoPermission,
    AuthorizationRevertFailed,
    AuthorizationRequiredInUpdate,

    CreateUserFailed = 10300,
    UsernameAlreadyContain,
    UpdateUserFailed,
    UsernameNotFound,
    UserNotFound,
    PasswordNotMatch,

    PostNotFound = 10400,
    TagsExceedMaximum,
    
    CategoryNotFound = 10500,
    CategoryAlreadyContain,

    GroupAlreadyContain = 10600,
    GroupNotFound,

    SaveFileFailed = 10700,
    StaticFileNotFound,
    
    CommentNotFound = 10800,
    SamePostCommentRequired,
    ReplyCommentRequired,
    SameParentCommentRequired,
    ReplyCommentMissing,

    IllegalText = 10900,
    EmailAlreadyContain,
    TooManyTags,
    UneditableTime,

    BannedStatus = 11000,
    CategoryArchived,
    CategoryStopped,
    PostArchived,
}