import * as React from 'react';
import * as ReactDOM from 'react-dom/client';
import { StyledEngineProvider } from '@mui/material/styles';
import Controla from './controla';
import dayjs from 'dayjs';
import 'dayjs/locale/es';

dayjs.locale('es');


ReactDOM.createRoot(document.querySelector("#root")!).render(
  <React.StrictMode>
    <StyledEngineProvider injectFirst>
      <Controla />
    </StyledEngineProvider>
  </React.StrictMode>
);