import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import useLocalStorage from '../../src/hooks/useLocalStorage';

describe('useLocalStorage', () => {
    beforeEach(() => {
        localStorage.clear();
        vi.clearAllMocks();
    });

    it('returns initial value when localStorage is empty', () => {
        const { result } = renderHook(() => useLocalStorage('testKey', 'default'));
        expect(result.current[0]).toBe('default');
    });

    it('returns stored value from localStorage', () => {
        localStorage.setItem('testKey', JSON.stringify('stored'));
        const { result } = renderHook(() => useLocalStorage('testKey', 'default'));
        expect(result.current[0]).toBe('stored');
    });

    it('updates localStorage when value changes', () => {
        const { result } = renderHook(() => useLocalStorage('testKey', 'initial'));

        act(() => {
            result.current[1]('updated');
        });

        expect(result.current[0]).toBe('updated');
        expect(JSON.parse(localStorage.getItem('testKey')!)).toBe('updated');
    });

    it('supports function updater', () => {
        const { result } = renderHook(() => useLocalStorage('counter', 0));

        act(() => {
            result.current[1]((prev) => prev + 1);
        });

        expect(result.current[0]).toBe(1);
    });

    it('removes value from localStorage', () => {
        localStorage.setItem('testKey', JSON.stringify('value'));
        const { result } = renderHook(() => useLocalStorage('testKey', 'default'));

        act(() => {
            result.current[2](); // removeValue
        });

        expect(result.current[0]).toBe('default');
        expect(localStorage.getItem('testKey')).toBeNull();
    });

    it('handles object values', () => {
        const initialValue = { name: 'test', count: 0 };
        const { result } = renderHook(() => useLocalStorage('objKey', initialValue));

        act(() => {
            result.current[1]({ name: 'updated', count: 5 });
        });

        expect(result.current[0]).toEqual({ name: 'updated', count: 5 });
    });
});
