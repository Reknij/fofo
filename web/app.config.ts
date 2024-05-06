export default defineAppConfig({
  ui: {
    primary: 'violet',
    gray: 'cool',
    card: {
      shadow: 'shadow-md',
      rounded: 'rounded',
      body: {
        padding: 'p-1 sm:p-2',
      },
      header: {
        padding: 'p-1 sm:p-2',
      },
      footer: {
        padding: 'p-1 sm:p-2',
      }
    },
    pagination: {
      rounded: 'first:rounded-s last:rounded-e',
    },
    badge: {
      rounded: 'rounded',
    },
    alert: {
      rounded: 'rounded',
      shadow: 'shadow-md'
    },
    button: {
      rounded: 'rounded',
      base: 'shadow-md',
      variant: {
        link: 'shadow-none'
      }
    },
    select: {
      rounded: 'rounded',
      base: "shadow-md"
    },
    selectMenu: {
      rounded: 'rounded',
      base: "shadow-md"
    },
    input: {
      rounded: 'rounded'
    },
    modal: {
      rounded: 'rounded'
    },
    dropdown: {
      rounded: 'rounded',
    },
  }
})