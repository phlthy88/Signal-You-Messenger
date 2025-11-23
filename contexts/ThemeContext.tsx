import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';

// Predefined M3-like seeds. In a full implementation, we'd use a color math library to generate palettes.
const THEMES = [
  {
    name: "Deep Purple",
    colors: {
      "--md-sys-color-primary": "#6750A4",
      "--md-sys-color-on-primary": "#FFFFFF",
      "--md-sys-color-primary-container": "#EADDFF",
      "--md-sys-color-on-primary-container": "#21005D",
      "--md-sys-color-secondary-container": "#E8DEF8",
      "--md-sys-color-on-secondary-container": "#1D192B",
      "--md-sys-color-surface": "#FFFBFE",
      "--md-sys-color-surface-variant": "#E7E0EC",
      "--md-sys-color-outline": "#79747E",
    }
  },
  {
    name: "Forest Green",
    colors: {
      "--md-sys-color-primary": "#006E1C",
      "--md-sys-color-on-primary": "#FFFFFF",
      "--md-sys-color-primary-container": "#94F990",
      "--md-sys-color-on-primary-container": "#002204",
      "--md-sys-color-secondary-container": "#DDE5D9",
      "--md-sys-color-on-secondary-container": "#161D17",
      "--md-sys-color-surface": "#FDFDF5",
      "--md-sys-color-surface-variant": "#DEE5D9",
      "--md-sys-color-outline": "#727970",
    }
  },
  {
    name: "Ocean Blue",
    colors: {
      "--md-sys-color-primary": "#0061A4",
      "--md-sys-color-on-primary": "#FFFFFF",
      "--md-sys-color-primary-container": "#D1E4FF",
      "--md-sys-color-on-primary-container": "#001D36",
      "--md-sys-color-secondary-container": "#D6E3FF",
      "--md-sys-color-on-secondary-container": "#101C2B",
      "--md-sys-color-surface": "#FDFBFF",
      "--md-sys-color-surface-variant": "#E0E2EC",
      "--md-sys-color-outline": "#74777F",
    }
  },
  {
    name: "Dark Mode",
    colors: {
      "--md-sys-color-primary": "#D0BCFF",
      "--md-sys-color-on-primary": "#381E72",
      "--md-sys-color-primary-container": "#4F378B",
      "--md-sys-color-on-primary-container": "#EADDFF",
      "--md-sys-color-secondary-container": "#4A4458",
      "--md-sys-color-on-secondary-container": "#E8DEF8",
      "--md-sys-color-surface": "#1C1B1F",
      "--md-sys-color-surface-variant": "#49454F",
      "--md-sys-color-on-surface": "#E6E1E5",
      "--md-sys-color-on-surface-variant": "#CAC4D0",
      "--md-sys-color-outline": "#938F99",
      "--md-sys-color-background": "#1C1B1F",
      "--md-sys-color-on-background": "#E6E1E5",
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
  const [currentTheme, setCurrentTheme] = useState("Deep Purple");

  useEffect(() => {
    const theme = THEMES.find(t => t.name === currentTheme);
    if (theme) {
      const root = document.documentElement;
      Object.entries(theme.colors).forEach(([property, value]) => {
        root.style.setProperty(property, value);
      });
    }
  }, [currentTheme]);

  return (
    <ThemeContext.Provider value={{ currentTheme, setTheme: setCurrentTheme, availableThemes: THEMES }}>
      {children}
    </ThemeContext.Provider>
  );
};

export const useTheme = () => {
  const context = useContext(ThemeContext);
  if (!context) throw new Error("useTheme must be used within a ThemeProvider");
  return context;
};