import React, { useState, forwardRef, useImperativeHandle } from 'react';
import {
  Grid,
  FormControl,
  FormControlLabel,
  Checkbox,
} from '@mui/material';
import { DatePicker } from '@mui/x-date-pickers/DatePicker';
import dayjs, { Dayjs } from 'dayjs';
import LocalizationProviderES from '../theme/location';

interface SelectorFechasFormData {
  ultimosRegistros: boolean;
  fechaInicio: Dayjs;
  fechaFin: Dayjs;
}

interface SelectorFechasProps {
  labelUltimosRegistros: string;
}

export interface SelectorFechasRef {
  getFormData: () => {
    fechaInicio: Dayjs | null;
    fechaFin: Dayjs | null;
  };
  setFormData: (newData: Partial<SelectorFechasFormData>) => void;
  resetForm: () => void;
}

// Componente selector que permitie elegir entre ver los
// últimos registros o elegir entre dos fechas
// con ref paa acceder a los valores desde el padre.
// Debe estar integrado en un grid.
export const SelectorFechas =
  forwardRef<SelectorFechasRef, SelectorFechasProps>((props, ref) => {
    const [formData, setFormData] = useState<SelectorFechasFormData>({
      ultimosRegistros: true,
      fechaInicio: dayjs(),
      fechaFin: dayjs()
    });

    const handleFieldChange =
      (field: keyof SelectorFechasFormData, value: boolean | Dayjs) => {
        setFormData(prev => ({
          ...prev,
          [field]: value
        }));
      };

    // Exponer método para obtener los valores desde el componente padre
    useImperativeHandle(ref, () => ({
      getFormData: () => {
        let fechaInicio = null;
        let fechaFin = null;

        if (!formData.ultimosRegistros) {
          fechaInicio = formData.fechaInicio;
          fechaFin = formData.fechaFin;
        }

        return {
          fechaInicio: fechaInicio,
          fechaFin: fechaFin
        }
      },
      setFormData: (newData) => setFormData(prev => ({ ...prev, ...newData })),
      resetForm: () => setFormData({
        ultimosRegistros: true,
        fechaInicio: dayjs(),
        fechaFin: dayjs()
      })
    }));

    return (
      <>
        <LocalizationProviderES>
          <Grid size={{ xs: 12, sm: 12, md: 2 }} sx={{ mt: 1 }}>
            <FormControl>
              <FormControlLabel
                name="ultimosRegistros"
                control={
                  <Checkbox
                    size="large"
                    checked={formData.ultimosRegistros}
                    onChange={(event: React.ChangeEvent<HTMLInputElement>) =>
                      handleFieldChange(
                        'ultimosRegistros', event.target.checked)}
                  />
                }
                label={props.labelUltimosRegistros}
              />
            </FormControl>
          </Grid>
          <Grid size={{ xs: 12, sm: 12, md: 2 }}>
            <DatePicker
              name='fechaInicio'
              label="Fecha inicio"
              value={formData.fechaInicio || null}
              disabled={formData.ultimosRegistros}
              onChange={(value: Dayjs | null) =>
                handleFieldChange('fechaInicio', value || dayjs())}
              sx={{ width: '100%' }}
            />
          </Grid>
          <Grid size={{ xs: 12, sm: 12, md: 2 }}>
            <DatePicker
              name='fechaFin'
              label="Fecha fin"
              value={formData.fechaFin || null}
              disabled={formData.ultimosRegistros}
              onChange={(value: Dayjs | null) =>
                handleFieldChange('fechaFin', value || dayjs())}
              sx={{ width: '100%' }}
            />
          </Grid>
        </LocalizationProviderES>
      </>
    );
  });