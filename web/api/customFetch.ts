import { AUTH_COOKIE_NAME } from "~/states/auth";

type useFetchType = typeof useFetch;

export const useApiFetch: useFetchType = (path, opts = {}) => {
  const config = useRuntimeConfig();
  const headers = useRequestHeaders([
    "x-bypass-key",
    "x-forwarded-for",
    "x-authorization",
  ]);
  const auth = useCookie(AUTH_COOKIE_NAME);
  if (auth.value) {
    headers["x-authorization"] = auth.value;
  }
  const baseURL = import.meta.client ? config.public.baseUrl : config.baseUrl;
  opts.baseURL = `${baseURL}/api`;
  opts.headers = headers;
  opts.key = `${path}${JSON.stringify(opts.query)}`;
  // opts.credentials = 'include'; // cloudflare workers will broken.
  return useFetch(path, opts);
};
