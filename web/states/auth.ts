import { loginUser, logoutUser, revertUser } from "~/api/user";
import { addDays, getApiDetailError, getServerInfoOnce } from "~/helper";
import type { ToLoginUser, UserInfo } from "~/models/user";
import type { ApiDetailError, VerificationTargetWrapper } from "~/models/util";
import cookie from "js-cookie";
import { useServerInfo } from "./server";

export const useCurrentUser = () =>
  useState<UserInfo | null>("currentUser", () => null);
export const useCurrentUserError = () =>
  useState<ApiDetailError | null>("currentUserError", () => null);

export const AUTH_COOKIE_NAME = "authorization";

function clearLastError() {
  const error = useCurrentUserError();
  error.value = null;
}
export function isLogined() {
  const currentUser = useCurrentUser().value;
  return currentUser != null;
}

export function getAuth() {
  return cookie.get(AUTH_COOKIE_NAME);
}

async function setAuth(auth: string) {
  const serverInfo = await getServerInfoOnce();
  cookie.set(AUTH_COOKIE_NAME, auth, {
    expires: serverInfo.value?.auth_active_days ?? 1,
  });
}

// Call it only in the nuxt context.
export async function revertInsideNuxt(manuallyAuth?: string): Promise<boolean> {
  const auth = useCookie(AUTH_COOKIE_NAME);
  if (manuallyAuth) auth.value = manuallyAuth;

  const currentUser = useCurrentUser();
  const fetchError = useCurrentUserError();

  if (auth.value) {
    try {
      const { data: current } = await revertUser(auth.value);
      currentUser.value = current.value;
      if (currentUser.value) return true;
      else {
        fetchError.value = {
          msg: "returned value is undefined",
          code: 0,
        };
        cookie.remove(AUTH_COOKIE_NAME);
      }
    } catch (error: any) {
      let data = error?.response?.data;
      fetchError.value = {
        code: data?.code ?? 0,
        msg: data?.msg ?? data,
      };
      cookie.remove(AUTH_COOKIE_NAME);
      currentUser.value = null;
    }
  }

  return false;
}

export async function login(
  q: VerificationTargetWrapper<ToLoginUser>
): Promise<boolean> {
  const currentUser = useCurrentUser();
  const fetchError = useCurrentUserError();
  clearLastError();
  let { data: anu, error } = await loginUser(q);
  if (anu.value) {
    currentUser.value = anu.value.user;
    await setAuth(anu.value.auth);
    return true;
  } else if (error.value) {
    const err = getApiDetailError(error.value);
    if (err) fetchError.value = err;
    cookie.remove(AUTH_COOKIE_NAME);
  }
  return false;
}

export async function logout(): Promise<boolean> {
  const currentUser = useCurrentUser();
  const fetchError = useCurrentUserError();
  clearLastError();
  let auth = cookie.get(AUTH_COOKIE_NAME);
  if (auth) {
    let { error } = await logoutUser(auth);
    if (error.value) {
      const err = getApiDetailError(error.value);
      if (err) fetchError.value = err;
    }
    currentUser.value = null;
    cookie.remove(AUTH_COOKIE_NAME);
  } else {
    fetchError.value = {
      code: -1,
      msg: "No authorization saved.",
    };
    return false;
  }
  return false;
}
