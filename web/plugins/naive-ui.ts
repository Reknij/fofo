import { setup } from "@css-render/vue3-ssr"
export default defineNuxtPlugin((nuxtApp) => {
  if (process.server) {
    const { collect } = setup(nuxtApp.vueApp)
    const originalRenderMeta = nuxtApp.ssrContext?.renderMeta
    // @ts-ignore
    nuxtApp.ssrContext = nuxtApp.ssrContext || {}
    // @ts-ignore
    nuxtApp.ssrContext.renderMeta = () => {
      if (!originalRenderMeta) {
        return {
          headTags: collect(),
        }
      }
      const originalMeta = originalRenderMeta()
      if ("then" in originalMeta) {
        return originalMeta.then((resolvedOriginalMeta: any) => {
          return {
            ...resolvedOriginalMeta,
            headTags: resolvedOriginalMeta.headTags + collect(),
          }
        })
      } else {
        return {
          ...originalMeta,
          headTags: originalMeta.headTags + collect(),
        }
      }
    }
    // @ts-ignore
    nuxtApp.ssrContext.head = nuxtApp.ssrContext.head || []
    // @ts-ignore
    nuxtApp.ssrContext.head.push({
      style: () =>
        collect()
          .split("</style>")
          .map((block) => {
            const id = RegExp(/cssr-id="(.+?)"/).exec(block)?.[1]
            const style = (RegExp(/>(.*)/s).exec(block)?.[1] ?? "").trim()
            return {
              "cssr-id": id,
              innerHTML: style,
            }
          }),
    })
  }
})
