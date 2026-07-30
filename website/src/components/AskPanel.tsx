import { useState, useRef } from 'react';
import { askBrain } from '../api';

export function AskPanel() {
  const [question, setQuestion] = useState('');
  const [answer, setAnswer] = useState('Answers stay on your local daemon.');
  const [loading, setLoading] = useState(false);
  const abortRef = useRef<AbortController | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!question.trim() || loading) return;
    abortRef.current?.abort();
    const ac = new AbortController();
    abortRef.current = ac;
    setLoading(true);
    setAnswer('Thinking…');
    try {
      const result = await askBrain(question.trim(), ac.signal);
      setAnswer(result);
    } catch (err) {
      if ((err as Error).name !== 'AbortError') {
        setAnswer('The daemon could not answer that question.');
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <section className="panel ask-panel" aria-labelledby="ask-heading">
      <div className="panel-heading">
        <p className="eyebrow">Synthesize</p>
        <h2 id="ask-heading">Ask the brain</h2>
      </div>
      <form id="ask-form" className="ask-form" onSubmit={handleSubmit}>
        <label className="sr-only" htmlFor="ask-input">Ask a question about your memory</label>
        <textarea
          id="ask-input"
          rows={3}
          placeholder="What connects these memories?"
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          disabled={loading}
        />
        <button type="submit" className="primary-button" disabled={loading || !question.trim()}>
          Ask <span aria-hidden="true">↗</span>
        </button>
      </form>
      <div id="ask-output" className="ask-output" aria-live="polite">
        {answer}
      </div>
    </section>
  );
}
