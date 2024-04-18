import { AUTH_COOKIE_NAME, isLogined, revertInsideNuxt } from "~/states/auth"

export default defineNuxtRouteMiddleware(async (to, from) => {
  if (useCookie(AUTH_COOKIE_NAME).value && !isLogined()) {
    let reverted = await revertInsideNuxt();
    if (to.path == '/login' && reverted) {
      return navigateTo('/')
    }
  }
})