import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import useDebounce from '../../src/hooks/useDebounce';

describe('useDebounce', () => {
    beforeEach(() => {
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    it('returns initial value immediately', () => {
        const { result } = renderHook(() => useDebounce('initial', 300));
        expect(result.current).toBe('initial');
    });

    it('debounces value updates', () => {
        const { result, rerender } = renderHook(
            ({ value }) => useDebounce(value, 300),
            { initialProps: { value: 'initial' } }
        );

        expect(result.current).toBe('initial');

        rerender({ value: 'updated' });
        expect(result.current).toBe('initial'); // Still initial

        act(() => {
            vi.advanceTimersByTime(299);
        });
        expect(result.current).toBe('initial'); // Still initial

        act(() => {
            vi.advanceTimersByTime(1);
        });
        expect(result.current).toBe('updated'); // Now updated
    });

    it('cancels previous debounce on new value', () => {
        const { result, rerender } = renderHook(
            ({ value }) => useDebounce(value, 300),
            { initialProps: { value: 'first' } }
        );

        rerender({ value: 'second' });
        act(() => {
            vi.advanceTimersByTime(200);
        });

        rerender({ value: 'third' });
        act(() => {
            vi.advanceTimersByTime(200);
        });

        expect(result.current).toBe('first'); // Still first

        act(() => {
            vi.advanceTimersByTime(100);
        });
        expect(result.current).toBe('third'); // Now third (skipped second)
    });
});
