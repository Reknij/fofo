import type {
  GetPresignedUrlQuery,
  GetPresignedUrlResult,
} from "~/models/storage_service";
import { useApiFetch } from "./customFetch";
import cookie from "js-cookie";
import { AUTH_COOKIE_NAME } from "~/states/auth";

export function getPresignPutUrl(query: GetPresignedUrlQuery) {
  return useApiFetch<GetPresignedUrlResult>(`/presign_put_url`, {
    query,
    key: `getPresignPutUrl${query}`,
  });
}

export function uploadFileToServer(presignUrl: string, blob: ArrayBuffer) {
  let headers: any = {};
  let auth = cookie.get(AUTH_COOKIE_NAME);
  if (auth) {
    headers["x-authorization"] = auth;
  }
  return useFetch(presignUrl, {
    method: "put",
    body: blob,
    headers,
    key: `uploadFile${presignUrl}`,
  });
}
