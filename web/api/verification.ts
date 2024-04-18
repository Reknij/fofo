import type { VerificationKeyPicture } from "~/models/verification";
import { useApiFetch } from "./customFetch";

export function getVerification() {
    return useApiFetch<VerificationKeyPicture>(`/verification`)
}