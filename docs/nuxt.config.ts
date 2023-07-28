export default defineNuxtConfig({
  // https://github.com/nuxt-themes/docus
  extends: "@nuxt-themes/docus",

  components: [
    { path: "./components/content/", global: true, prefix: "" },
    { path: "./components/", global: true, prefix: "" },
  ],

  modules: [
    // https://github.com/nuxt-modules/plausible
    "@nuxtjs/plausible",
    // https://github.com/nuxt/devtools
    // '@nuxt/devtools'
  ],
});
