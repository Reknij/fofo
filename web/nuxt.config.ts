// https://nuxt.com/docs/api/configuration/nuxt-confign
export default defineNuxtConfig({
  modules: ["@nuxtjs/device", "nuxt-jsonld", "@nuxt/ui", "@nuxt/image"],
  ui: {
    icons: ['mdi']
  },
  routeRules: {
  },
  runtimeConfig: {
    public: {
      baseUrl: "",
      externalLinks: [
        {
          label: "GitHub",
          href: "https://github.com/Reknij/fofo",
          icon: "i-mdi-github"
        },
      ],
      forumName: "Fofo",
      limitData: {
        any: 20,
        comments: 10,
      },
      default: {
        distinct: false,
      },
    },
    baseUrl: "http://0.0.0.0:6688",
  },

  nitro: {
    devProxy: {
      "/api": {
        target: "http://0.0.0.0:6688/api",
      },
    },
  },
});