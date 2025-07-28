import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface User {
  id: string;
  username: string;
  email: string;
  password_hash: string;
  is_admin: boolean;
  created_at: string;
  updated_at: string;
}

interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (token: string, user: User) => void;
  logout: () => void;
  setLoading: (loading: boolean) => void;
  initializeAuth: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      token: null,
      isAuthenticated: false,
      isLoading: false,
      login: (token: string, user: User) => {
        document.cookie = `auth-token=${token}; path=/; max-age=${24 * 60 * 60}; SameSite=Lax`;
        set({ token, user, isAuthenticated: true, isLoading: false });
      },
      logout: () => {
        document.cookie = 'auth-token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT';
        set({ token: null, user: null, isAuthenticated: false, isLoading: false });
      },
      setLoading: (loading: boolean) => {
        set({ isLoading: loading });
      },
      initializeAuth: () => {
        const state = get();
        const cookieToken = document.cookie
          .split('; ')
          .find(row => row.startsWith('auth-token='))
          ?.split('=')[1];
        
        if (cookieToken && state.token && state.user) {
          set({ isAuthenticated: true });
        } else if (!cookieToken) {
          set({ token: null, user: null, isAuthenticated: false });
        } else if (cookieToken && (!state.token || !state.user)) {
          set({ token: null, user: null, isAuthenticated: false });
          document.cookie = 'auth-token=; path=/; expires=Thu, 01 Jan 1970 00:00:00 GMT';
        }
      },
    }),
    {
      name: 'auth-storage',
      partialize: (state: AuthState) => ({ 
        token: state.token, 
        user: state.user, 
        isAuthenticated: state.isAuthenticated 
      }),
    }
  )
);