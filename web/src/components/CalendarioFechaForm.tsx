import * as React from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import FormControl from '@mui/material/FormControl';
import FormHelperText from '@mui/material/FormHelperText';
import Grid from '@mui/material/Grid';
import InputLabel from '@mui/material/InputLabel';
import MenuItem from '@mui/material/MenuItem';
import Select, { SelectChangeEvent } from '@mui/material/Select';
import Stack from '@mui/material/Stack';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import { Dayjs } from 'dayjs';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import { useIsMounted } from '../hooks/useComponentMounted';
import { DatePicker } from '@mui/x-date-pickers/DatePicker';
import LocalizationProviderES from '../theme/location';
import { CalendarioFecha, NombresTipoCalendarioFecha, TipoCalendarioFecha } from '../modelos/calendario';
import { validarFechaHora } from '../error';

export interface CalendarioFechaFormState {
  values: Partial<CalendarioFecha>;
  errors: Partial<Record<keyof CalendarioFecha, string>>;
}

export type FormFieldValue = string | number | Dayjs | null;

export interface CalendarioFechaFormProps {
  formState: CalendarioFechaFormState;
  onFieldChange: (
    name: keyof CalendarioFechaFormState['values'],
    value: FormFieldValue,
  ) => void;
  onSubmit: (formValues: Partial<CalendarioFechaFormState['values']>) => Promise<void>;
  onReset?: () => void;
  submitButtonLabel: string;
  onBack: () => void;
}

export type ValidationResult = {
  issues: { message: string; path: (keyof CalendarioFecha)[] }[]
};

export function validaCalendarioFecha(fecha: Partial<CalendarioFecha>): ValidationResult {
  const issues: ValidationResult['issues'] = [];

  if (!fecha.fechaInicio || !validarFechaHora(fecha.fechaInicio)) {
    issues.push({ message: 'La fecha de inicio es requerida', path: ['fechaInicio'] });
  }

  if (!fecha.fechaFin || !validarFechaHora(fecha.fechaFin)) {
    issues.push({ message: 'La fecha de fin es requerida', path: ['fechaFin'] });
  }

  if (fecha.fechaInicio && fecha.fechaFin && validarFechaHora(fecha.fechaInicio) && validarFechaHora(fecha.fechaFin)) {
    if (fecha.fechaFin.isBefore(fecha.fechaInicio)) {
      issues.push({ message: 'La fecha fin debe ser mayor o igual a la fecha inicio', path: ['fechaFin'] });
    }
  }

  return { issues };
}

export default function CalendarioFechaForm(props: CalendarioFechaFormProps) {
  const {
    formState,
    onFieldChange,
    onSubmit,
    onReset,
    submitButtonLabel,
    onBack,
  } = props;

  const isMounted = useIsMounted();
  const { values: formValues, errors: formErrors } = formState;
  const [isSubmitting, setIsSubmitting] = React.useState(false);

  const handleSubmit = React.useCallback(
    async (event: React.FormEvent<HTMLFormElement>) => {
      event.preventDefault();
      setIsSubmitting(true);
      try {
        await onSubmit(formValues);
      } finally {
        if (isMounted.current) {
          setIsSubmitting(false);
        }
      }
    },
    [formValues, onSubmit, isMounted],
  );

  const handleSelectFieldChange = React.useCallback(
    (event: SelectChangeEvent<any>) => {
      onFieldChange(
        event.target.name as keyof CalendarioFechaFormState['values'],
        event.target.value,
      );
    },
    [onFieldChange],
  );

  const handleDateFieldChange = React.useCallback(
    (name: keyof CalendarioFechaFormState['values']) => (value: Dayjs | null) => {
      onFieldChange(name, value);
    },
    [onFieldChange],
  );

  const tipoOptions = React.useMemo(() => {
    return Object.entries(TipoCalendarioFecha).map(([_, value]) => ({
      label: NombresTipoCalendarioFecha[value as TipoCalendarioFecha],
      value: value
    }));
  }, []);

  return (
    <Box
      component="form"
      onSubmit={handleSubmit}
      noValidate
      autoComplete="off"
      onReset={onReset}
      sx={FULL_HEIGHT_WIDTH}
    >
      <LocalizationProviderES>
        <Grid container spacing={2} sx={{ mt: 2, mb: 4, width: '100%' }}>
          <Grid size={{ xs: 12, sm: 6 }}>
            <DatePicker
              label="Fecha inicio"
              value={formValues.fechaInicio ?? null}
              onChange={handleDateFieldChange('fechaInicio')}
              slotProps={{
                textField: {
                  fullWidth: true,
                  error: !!formErrors.fechaInicio,
                  helperText: formErrors.fechaInicio ?? ' '
                }
              }}
            />
          </Grid>
          <Grid size={{ xs: 12, sm: 6 }}>
            <DatePicker
              label="Fecha fin"
              value={formValues.fechaFin ?? null}
              onChange={handleDateFieldChange('fechaFin')}
              slotProps={{
                textField: {
                  fullWidth: true,
                  error: !!formErrors.fechaFin,
                  helperText: formErrors.fechaFin ?? ' '
                }
              }}
            />
          </Grid>
          <Grid size={{ xs: 12 }}>
            <FormControl error={!!formErrors.tipo} fullWidth>
              <InputLabel>Tipo</InputLabel>
              <Select
                name="tipo"
                value={formValues.tipo}
                label="Tipo"
                onChange={handleSelectFieldChange}
              >
                {tipoOptions.map((option) => (
                  <MenuItem key={option.value} value={option.value}>
                    {option.label}
                  </MenuItem>
                ))}
              </Select>
              <FormHelperText>{formErrors.tipo ?? ' '}</FormHelperText>
            </FormControl>
          </Grid>
        </Grid>
      </LocalizationProviderES>
      <Stack direction="row" spacing={2} justifyContent="space-between">
        <Button variant="contained" startIcon={<ArrowBackIcon />} onClick={onBack}>
          VOLVER
        </Button>
        <Button type="submit" variant="contained" size="large" loading={isSubmitting}>
          {submitButtonLabel}
        </Button>
      </Stack>
    </Box>
  );
}