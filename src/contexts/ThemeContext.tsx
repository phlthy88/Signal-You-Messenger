import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { useStore } from '../store';

const THEMES = [
  {
    name: "Deep Purple",
    colors: {
      "--md-sys-color-primary": "#6750A4",
      "--md-sys-color-on-primary": "#FFFFFF",
      "--md-sys-color-primary-container": "#EADDFF",
      "--md-sys-color-on-primary-container": "#21005D",
      "--md-sys-color-secondary": "#625B71",
      "--md-sys-color-on-secondary": "#FFFFFF",
      "--md-sys-color-secondary-container": "#E8DEF8",
      "--md-sys-color-on-secondary-container": "#1D192B",
      "--md-sys-color-tertiary": "#7D5260",
      "--md-sys-color-on-tertiary": "#FFFFFF",
      "--md-sys-color-tertiary-container": "#FFD8E4",
      "--md-sys-color-on-tertiary-container": "#31111D",
      "--md-sys-color-surface": "#FFFBFE",
      "--md-sys-color-surface-variant": "#E7E0EC",
      "--md-sys-color-on-surface": "#1C1B1F",
      "--md-sys-color-on-surface-variant": "#49454F",
      "--md-sys-color-outline": "#79747E",
      "--md-sys-color-outline-variant": "#CAC4D0",
      "--md-sys-color-background": "#FFFBFE",
      "--md-sys-color-on-background": "#1C1B1F",
      "--md-sys-color-surface-container": "#F3EDF7",
    }
  },
  {
    name: "Forest Green",
    colors: {
      "--md-sys-color-primary": "#006E1C",
      "--md-sys-color-on-primary": "#FFFFFF",
      "--md-sys-color-primary-container": "#94F990",
      "--md-sys-color-on-primary-container": "#002204",
      "--md-sys-color-secondary": "#52634F",
      "--md-sys-color-on-secondary": "#FFFFFF",
      "--md-sys-color-secondary-container": "#DDE5D9",
      "--md-sys-color-on-secondary-container": "#161D17",
      "--md-sys-color-tertiary": "#386666",
      "--md-sys-color-on-tertiary": "#FFFFFF",
      "--md-sys-color-tertiary-container": "#BBEBEB",
      "--md-sys-color-on-tertiary-container": "#002020",
      "--md-sys-color-surface": "#FDFDF5",
      "--md-sys-color-surface-variant": "#DEE5D9",
      "--md-sys-color-on-surface": "#1A1C18",
      "--md-sys-color-on-surface-variant": "#43483F",
      "--md-sys-color-outline": "#727970",
      "--md-sys-color-outline-variant": "#C2C9BD",
      "--md-sys-color-background": "#FDFDF5",
      "--md-sys-color-on-background": "#1A1C18",
      "--md-sys-color-surface-container": "#EEEEE6",
    }
  },
  {
    name: "Ocean Blue",
    colors: {
      "--md-sys-color-primary": "#0061A4",
      "--md-sys-color-on-primary": "#FFFFFF",
      "--md-sys-color-primary-container": "#D1E4FF",
      "--md-sys-color-on-primary-container": "#001D36",
      "--md-sys-color-secondary": "#535F70",
      "--md-sys-color-on-secondary": "#FFFFFF",
      "--md-sys-color-secondary-container": "#D6E3FF",
      "--md-sys-color-on-secondary-container": "#101C2B",
      "--md-sys-color-tertiary": "#6B5778",
      "--md-sys-color-on-tertiary": "#FFFFFF",
      "--md-sys-color-tertiary-container": "#F2DAFF",
      "--md-sys-color-on-tertiary-container": "#251432",
      "--md-sys-color-surface": "#FDFBFF",
      "--md-sys-color-surface-variant": "#E0E2EC",
      "--md-sys-color-on-surface": "#1A1C1E",
      "--md-sys-color-on-surface-variant": "#44474E",
      "--md-sys-color-outline": "#74777F",
      "--md-sys-color-outline-variant": "#C4C6D0",
      "--md-sys-color-background": "#FDFBFF",
      "--md-sys-color-on-background": "#1A1C1E",
      "--md-sys-color-surface-container": "#ECEDF4",
    }
  },
  {
    name: "Dark Mode",
    colors: {
      "--md-sys-color-primary": "#D0BCFF",
      "--md-sys-color-on-primary": "#381E72",
      "--md-sys-color-primary-container": "#4F378B",
      "--md-sys-color-on-primary-container": "#EADDFF",
      "--md-sys-color-secondary": "#CCC2DC",
      "--md-sys-color-on-secondary": "#332D41",
      "--md-sys-color-secondary-container": "#4A4458",
      "--md-sys-color-on-secondary-container": "#E8DEF8",
      "--md-sys-color-tertiary": "#EFB8C8",
      "--md-sys-color-on-tertiary": "#492532",
      "--md-sys-color-tertiary-container": "#633B48",
      "--md-sys-color-on-tertiary-container": "#FFD8E4",
      "--md-sys-color-surface": "#1C1B1F",
      "--md-sys-color-surface-variant": "#49454F",
      "--md-sys-color-on-surface": "#E6E1E5",
      "--md-sys-color-on-surface-variant": "#CAC4D0",
      "--md-sys-color-outline": "#938F99",
      "--md-sys-color-outline-variant": "#49454F",
      "--md-sys-color-background": "#1C1B1F",
      "--md-sys-color-on-background": "#E6E1E5",
      "--md-sys-color-surface-container": "#211F26",
    }
  }
];

interface ThemeContextType {
  currentTheme: string;
  setTheme: (name: string) => void;
  availableThemes: typeof THEMES;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export const ThemeProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
  const settings = useStore((state) => state.settings);
  const updateSettings = useStore((state) => state.updateSettings);
  const isAuthenticated = useStore((state) => state.isAuthenticated);

  const [currentTheme, setCurrentTheme] = useState(settings.theme || "Deep Purple");

  useEffect(() => {
    setCurrentTheme(settings.theme || "Deep Purple");
  }, [settings.theme]);

  useEffect(() => {
    const theme = THEMES.find(t => t.name === currentTheme);
    if (theme) {
      const root = document.documentElement;
      Object.entries(theme.colors).forEach(([property, value]) => {
        root.style.setProperty(property, value);
      });
    }
  }, [currentTheme]);

  const setTheme = async (name: string) => {
    setCurrentTheme(name);
    if (isAuthenticated) {
      try {
        await updateSettings({ theme: name });
      } catch (error) {
        console.error('Failed to save theme preference:', error);
      }
    }
  };

  return (
    <ThemeContext.Provider value={{ currentTheme, setTheme, availableThemes: THEMES }}>
      {children}
    </ThemeContext.Provider>
  );
};

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (!context) throw new Error("useTheme must be used within a ThemeProvider");
  return context;
};
