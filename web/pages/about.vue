<script setup lang="ts">
import { NCard, useLoadingBar } from 'naive-ui';

const config = useRuntimeConfig();

const loadingBar = useLoadingBar()
loadingBar.start();

onMounted(() => loadingBar.finish());
</script>

<template>
    <n-card>
        <h1>{{ config.public.forumName }}</h1>
        Fofo is a freedom forum. Here you can say what you want, but the following rules need to be met.
        <ul>
            <li>
                Do not verbally abuse others for no reason.
            </li>
            <li>
                Keep basic friendly exchanges.
            </li>
            <li>
                Unsubstantiated rumors cannot be knowingly spread.
            </li>
        </ul>
        <img style="max-width: 100%" src="https://upload.wikimedia.org/wikipedia/commons/thumb/7/79/Face-smile.svg/768px-Face-smile.svg.png" />
    </n-card>
</template>