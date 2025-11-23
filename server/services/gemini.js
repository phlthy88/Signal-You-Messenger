import { GoogleGenerativeAI } from '@google/generative-ai';
import config from '../config/env.js';

let genAI = null;

if (config.GEMINI_API_KEY) {
  genAI = new GoogleGenerativeAI(config.GEMINI_API_KEY);
}

export async function generateSmartReplies(messages, userId) {
  if (!genAI) {
    return { replies: [], error: 'AI service not configured' };
  }

  try {
    const model = genAI.getGenerativeModel({ model: 'gemini-1.5-flash' });

    const conversation = messages.slice(-10).map(m =>
      `${m.sender_id === userId ? 'Me' : m.sender_name}: ${m.content}`
    ).join('\n');

    const prompt = `Based on the following conversation, suggest 3 short, relevant, and conversational replies for "Me".
Return ONLY the replies, separated by a pipe character (|). No numbering, no explanations.

Conversation:
${conversation}`;

    const result = await model.generateContent(prompt);
    const text = result.response.text() || '';
    const replies = text.split('|').map(s => s.trim()).filter(s => s.length > 0 && s.length < 200);

    return { replies: replies.slice(0, 3) };
  } catch (error) {
    console.error('Gemini Smart Reply Error:', error.message);
    return { replies: [], error: 'Failed to generate replies' };
  }
}

export async function summarizeConversation(messages, userId) {
  if (!genAI) {
    return { summary: '', error: 'AI service not configured' };
  }

  try {
    const model = genAI.getGenerativeModel({ model: 'gemini-1.5-flash' });

    const conversation = messages.map(m =>
      `${m.sender_id === userId ? 'Me' : m.sender_name}: ${m.content}`
    ).join('\n');

    const prompt = `Summarize this conversation in one concise paragraph (2-3 sentences max):

${conversation}`;

    const result = await model.generateContent(prompt);
    const summary = result.response.text() || '';

    return { summary: summary.trim() };
  } catch (error) {
    console.error('Gemini Summary Error:', error.message);
    return { summary: '', error: 'Failed to generate summary' };
  }
}

export function isAIEnabled() {
  return !!genAI;
}
