export default defineAppConfig({
  docus: {
    title: 'Densky',
    description: 'Densky is the backend framework for Deno',
    image: 'https://user-images.githubusercontent.com/904724/185365452-87b7ca7b-6030-4813-a2db-5e65c785bf88.png',
    socials: {
      github: 'Densky-Framework/densky',
    },
    github: {
      dir: '.',
      branch: 'main',
      repo: 'densky',
      owner: 'Densky-Framework',
      edit: true
    },
    aside: {
      level: 0,
      collapsed: false,
      exclude: []
    },
    main: {
      padded: true,
      fluid: true
    },
    header: {
      logo: true,
      showLinkIcon: true,
      exclude: [],
      fluid: true
    },
    footer: {
      credits: false
    }
  }
})
