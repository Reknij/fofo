import { isLogined, revert } from "~/states/auth"

export default defineNuxtRouteMiddleware(async (to, from) => {
  if (!isLogined()) {
    let reverted = await revert();
    if (to.path == '/login' && reverted) {
      return navigateTo('/')
    }
  }
})