<script setup lang="ts">
import type { CommentInfo } from '~/models/comment';
import type { PostInfo } from '~/models/post';
import { ContentType } from '~/models/util';

defineProps<{
    info: PostInfo | CommentInfo
}>()

const modal = useModal();
</script>

<template>
    <UModal>
        <UCard :ui="{ ring: '', divide: 'divide-y divide-gray-100 dark:divide-gray-800' }">
            <template #header>
                <div class="flex items-center gap-x-1 justify-between">
                    <h3 class="text-base font-semibold leading-6 text-gray-900 dark:text-white">
                        ID {{ info.id }}
                    </h3>
                    <UButton color="gray" variant="ghost" icon="i-heroicons-x-mark-20-solid" class="-my-1"
                        @click="modal.close()" />
                </div>
            </template>

            <div class="space-y-2">
                <div class="flex items-center gap-x-1">
                    <span>Category id:</span>
                    <span class="code">{{ info.category_id }}</span>
                </div>
                <div class="flex items-center gap-x-1" v-if="(info as any).post_id">
                    <span>Post id:</span>
                    <span class="code">{{ (info as any).post_id }}</span>
                </div>
                <div class="flex items-center gap-x-1" v-if="(info as any).parent_id">
                    <span>Parent comment id:</span>
                    <span class="code">{{ (info as any).parent_id }}</span>
                </div>
                <div class="flex items-center gap-x-1">
                    <span>Created by id:</span>
                    <span class="code">{{ info.created_by_id }}</span>
                </div>
                <div class="flex items-center gap-x-1">
                    <span>Created at:</span>
                    <span class="code">{{ new Date(info.created_at * 1000).toString() }}</span>
                </div>
                <div class="flex items-center gap-x-1">
                    <span>Last edit by id:</span>
                    <span class="code">{{ info.last_edit_by_id }}</span>
                </div>
                <div class="flex items-center gap-x-1">
                    <span>Last edit at:</span>
                    <span class="code">{{ new Date(info.last_edit_at * 1000).toString() }}</span>
                </div>
                <div class="flex items-center gap-x-1">
                    <span>Content type:</span>
                    <span class="code">{{ ContentType[info.content_type] }}</span>
                </div>
                <div class="flex items-center gap-x-1">
                    <span>Likes:</span>
                    <span class="code">{{ info.likes }}</span>
                </div>
                <div class="flex items-center gap-x-1">
                    <span>Dislikes:</span>
                    <span class="code">{{ info.dislikes }}</span>
                </div>
                <div class="flex items-center gap-x-1">
                    <span>Total comment:</span>
                    <span class="code">{{ info.total_comment }}</span>
                </div>
                <div class="flex items-center gap-x-1">
                    <span>Last comment by id:</span>
                    <span class="code">{{ info.last_comment_by_id }}</span>
                </div>
                <div class="flex items-center gap-x-1">
                    <span>Last comment at:</span>
                    <span class="code">{{ new Date(info.last_comment_at * 1000).toString() }}</span>
                </div>
                <div class="flex items-center gap-x-1" v-if="(info as any).reply_user_id">
                    <span>Reply user id:</span>
                    <span class="code">{{ (info as any).reply_user_id }}</span>
                </div>
                <div class="flex items-center gap-x-1" v-if="(info as any).reply_comment_id">
                    <span>Reply comment id:</span>
                    <span class="code">{{ (info as any).reply_comment_id }}</span>
                </div>
                <div class="flex items-center gap-x-1">
                    <span>Top index:</span>
                    <span class="code">{{ info.top_index ? info.top_index : 'Unset' }}</span>
                </div>
            </div>
        </UCard>
    </UModal>
</template>