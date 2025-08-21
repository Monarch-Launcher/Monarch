import { invoke } from '@tauri-apps/api/core';
import type { ProtonVersion } from '@global/types';
import React, { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react';

interface ProtonVersionsContextType {
  protonVersions: ProtonVersion[];
  isLoading: boolean;
  error: string | null;
  refetch: () => Promise<void>;
}

const ProtonVersionsContext = createContext<ProtonVersionsContextType | undefined>(undefined);

interface ProtonVersionsProviderProps {
  children: React.ReactNode;
}

export const ProtonVersionsProvider: React.FC<ProtonVersionsProviderProps> = ({ children }) => {
  const [protonVersions, setProtonVersions] = useState<ProtonVersion[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchProtonVersions = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<ProtonVersion[]>('proton_versions');
      setProtonVersions(Array.isArray(result) ? result : []);
    } catch (err: any) {
      setError('Failed to load Proton versions');
      // Error already logged via setError
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchProtonVersions();
  }, [fetchProtonVersions]);

  const value: ProtonVersionsContextType = useMemo(
    () => ({
      protonVersions,
      isLoading,
      error,
      refetch: fetchProtonVersions,
    }),
    [protonVersions, isLoading, error, fetchProtonVersions],
  );

  return (
    <ProtonVersionsContext.Provider value={value}>
      {children}
    </ProtonVersionsContext.Provider>
  );
};

export const useProtonVersions = (): ProtonVersionsContextType => {
  const context = useContext(ProtonVersionsContext);
  if (context === undefined) {
    throw new Error('useProtonVersions must be used within a ProtonVersionsProvider');
  }
  return context;
};
