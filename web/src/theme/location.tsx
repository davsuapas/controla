// components/LocalizationProviderES.jsx
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';

export interface LocalizationProviderESProps {
  children?: React.ReactNode;
}

export default function LocalizationProviderES({
  children,
}: LocalizationProviderESProps) {
  return (
    <LocalizationProvider
      dateAdapter={AdapterDayjs}
      adapterLocale='es'
    >
      {children}
    </LocalizationProvider>
  );
};
