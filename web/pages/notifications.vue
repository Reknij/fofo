<script setup lang="ts">
import { useCurrentUser } from '~/states/auth';

const user = useCurrentUser();
const router = useRouter();
const links = [
    {
        label: 'Notifications',
    }
]

</script>

<template>
    <div>
        <div class="space-y-1.5" v-if="user">
            <FofoBreadcrumb :links="links"></FofoBreadcrumb>
            <UserNotifications />
        </div>
        <div class="space-y-1.5" v-else>
            <UCard>
                <template #header>
                    Please login first.
                </template>
                <UButton to="/login" label="Login now" />
            </UCard>
        </div>
    </div>
</template>