import type { Category } from "~/models/category";
import { type SafeUserInfo, UserType } from "~/models/user";
import type { ApiDetailError } from "~/models/util";
import Identicon from "identicon.js";
import gfm from "@bytemd/plugin-gfm";
import mathSsr from "@bytemd/plugin-math-ssr";
import mediumZoom from "@bytemd/plugin-medium-zoom";
import highlightSsr from "@bytemd/plugin-highlight-ssr";
import sha256 from "crypto-js/sha256";
import { useServerInfo } from "~/states/server";
import { getServerInfo } from "~/api/server";

// Define the type of the input and output of the function
export function timeAgo(timestampSeconds: number, withAgo = false): string {
  timestampSeconds *= 1000;
  // Convert date string to Date object
  const date = new Date(timestampSeconds);
  // Get current Date
  const now = new Date();
  // Calculate difference in milliseconds
  const diff = now.getTime() - date.getTime();
  const lastText = withAgo ? " ago" : "";
  // Format output string
  if (diff >= 365 * 24 * 60 * 60 * 1000) {
    const years = Math.floor(diff / (365 * 24 * 60 * 60 * 1000));
    return `${years} year(s)${lastText}`;
  } else if (diff >= 30 * 24 * 60 * 60 * 1000) {
    const months = Math.floor(diff / (30 * 24 * 60 * 60 * 1000));
    return `${months} month(s)${lastText}`;
  } else if (diff >= 24 * 60 * 60 * 1000) {
    const days = Math.floor(diff / (24 * 60 * 60 * 1000));
    return `${days} day(s)${lastText}`;
  } else if (diff >= 60 * 60 * 1000) {
    const hours = Math.floor(diff / (60 * 60 * 1000));
    return `${hours} hour(s)${lastText}`;
  } else if (diff >= 60 * 1000) {
    const minutes = Math.floor(diff / (60 * 1000));
    return `${minutes} minute(s)${lastText}`;
  } else if (diff >= 1000) {
    const seconds = Math.floor(diff / 1000);
    return `${seconds} second(s)${lastText}`;
  } else {
    return `just now`;
  }
}

export function addDays(date: Date, days: number) {
  var result = new Date(date);
  result.setDate(result.getDate() + days);
  return result;
}

export async function getServerInfoOnce() {
  const serverInfo = useServerInfo();
  if (!serverInfo.value) {
    const { data } = await getServerInfo();
    serverInfo.value = data.value;
  }
  return serverInfo;
}

export async function isEditable(create_at: number) {
  const now = Date.now() / 1000;
  const serverInfo = await getServerInfoOnce();
  if (serverInfo.value && serverInfo.value.editable_seconds + create_at > now) {
    return true;
  }
  return false;
}

export function parseAny<T>(any: any): T | undefined {
  if (any) return any as T;
  else return undefined;
}

function getPlugins() {
  return [
    gfm(),
    mathSsr(),
    mediumZoom(),
    highlightSsr(),
    // Add more plugins here
  ];
}

export const bytemdPlugins = getPlugins();

export function hasManagePermission(
  user: SafeUserInfo | undefined | null,
  category: Category
) {
  if (user) {
    if (user.user_type === UserType.Administrator) return true;

    let id = category.moderator_ids.find((id) => id == user.id);
    if (user.id === id) {
      return true;
    }
  }
  return false;
}

export function getAvatar(user: SafeUserInfo | undefined | null) {
  var t = "fofoavatargeneratedefault";
  if (user) {
    if (user.avatar_url) return user.avatar_url;
    t = `${user.username}_${user.id}`;
  }
  const hash = sha256(t);
  var data = new Identicon(hash.toString()).toString();
  const source = `data:image/png;base64,${data}`;
  return source;
}

export function getApiDetailError(error: any): ApiDetailError | undefined {
  if (error && error.data) {
    const err = error.data;
    const code = err.code;
    const msg = err.msg;
    if (code && msg)
      return {
        code,
        msg,
      };
    else
      return {
        code: -1,
        msg: err,
      };
  }

  return undefined;
}
