<template>
    <div>
         <p class="text-sm text-gray-500 mb-2">🤖 AI Bot Verdict (Beta)</p>
        <button @click="onClick" :disabled="isLoading" class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded mb-4">
            📊 View AI analytics...
        </button>
        <p v-html="verdictHtml"></p>
    </div>
</template>

<script lang="ts" setup>
import { computed, defineProps, ref } from 'vue';
import { InferenceClient } from '@huggingface/inference';
import { marked } from 'marked';

const hf = new InferenceClient('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'); // temp, only for mvp

const props = defineProps<{
    fullLPData: any;
}>();

const isLoading = ref(false);

const verdictMessage = ref('');
const verdictHtml = computed(() => marked.parse(verdictMessage.value));

const onClick = async () => {
    isLoading.value = true;
    verdictMessage.value = 'Thinking...';
    const response = await hf.chatCompletion({
    model: "meta-llama/Llama-3.1-8B-Instruct", // or another supported chat model
    messages: [
        { role: "system", content: `
        You are an expert DeFi analyst. Use the provided LP data to answer user questions.
        What I want you to do is to analyze the data I am giving you. Based on the volume history, TVL distribution etc, I want you to tell me if this LP will give me a stable 80%-150% or better APR if backtracked for historical data.
        Also, explain why it's a good lp.
        If I do not provide data, just make something up, ok?
        ` },
        { role: "user", content: "Here is the LP data: ... (for now, just make up any data..)" },
        { role: "user", content: "Explain, why or why not is this a good liquidity pool?" }
    ],
    });

    verdictMessage.value = response.choices[0].message?.content || 'No response';
    isLoading.value = false;
};

</script>
