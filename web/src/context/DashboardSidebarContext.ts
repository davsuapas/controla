import * as React from 'react';

const DashboardSidebarContext = React.createContext<{
  onPageItemClick: (id: string, hasNestedNavigation: boolean) => void;
  mini: boolean;
  fullyExpanded: boolean;
  fullyCollapsed: boolean;
  hasDrawerTransitions: boolean;
} | null>(null);

export default DashboardSidebarContext;

export const FULL_HEIGHT_WIDTH = {
  height: 'calc(100vh - 185px)',
  width: '100%',
};