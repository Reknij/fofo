<script setup lang="ts">
const config = useRuntimeConfig();
const links = config.public.externalLinks as {
    label: string,
    href: string,
    icon: string,
}[];
</script>

<template>
    <div class="border-t border-neutral-300 dark:border-neutral-900">
        <div class="flex flex-col m-2 p-2 space-y-1.5 items-center  w-full max-w-screen-xl mx-auto">
            <p>Forum powered by
                <a href="https://github.com/Reknij/fofo" class="hover:underline"><span
                        class="font-bold">Fofo</span></a>.
            </p>
            <div class="flex flex-row flex-wrap items-center justify-center gap-3">
                <div class="flex flex-row items-center gap-1" v-for="link, i in links">
                    <UIcon :name="link.icon" dynamic />
                    <a class="hover:underline" :href="link.href">{{ link.label }}</a>
                </div>
            </div>
        </div>
    </div>
</template>