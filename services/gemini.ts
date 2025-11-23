import { GoogleGenAI } from "@google/genai";
import { Message } from "../types";

const ai = new GoogleGenAI({ apiKey: process.env.API_KEY });

export const generateSmartReplies = async (messages: Message[]): Promise<string[]> => {
  if (!process.env.API_KEY) return [];

  // Format context for the AI
  const conversation = messages.slice(-10).map(m => 
    `${m.isMe ? 'Me' : 'Partner'}: ${m.content}`
  ).join('\n');

  const prompt = `
    Based on the following conversation, suggest 3 short, relevant, and conversational replies for "Me".
    Return ONLY the replies, separated by a pipe character (|).
    
    Conversation:
    ${conversation}
  `;

  try {
    const response = await ai.models.generateContent({
      model: 'gemini-2.5-flash',
      contents: prompt,
    });

    const text = response.text || '';
    return text.split('|').map(s => s.trim()).filter(s => s.length > 0);
  } catch (error) {
    console.error("Gemini Smart Reply Error:", error);
    return [];
  }
};

export const summarizeConversation = async (messages: Message[]): Promise<string> => {
  if (!process.env.API_KEY) return "API Key missing.";

  const conversation = messages.map(m => 
    `${m.isMe ? 'Me' : 'Partner'}: ${m.content}`
  ).join('\n');

  try {
    const response = await ai.models.generateContent({
      model: 'gemini-2.5-flash',
      contents: `Summarize this conversation in one concise paragraph: \n\n${conversation}`,
    });
    return response.text || "Could not generate summary.";
  } catch (error) {
    console.error("Gemini Summary Error:", error);
    return "Error generating summary.";
  }
};