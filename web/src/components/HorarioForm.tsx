import * as React from 'react';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import FormControl from '@mui/material/FormControl';
import FormHelperText from '@mui/material/FormHelperText';
import Grid from '@mui/material/Grid';
import InputLabel from '@mui/material/InputLabel';
import MenuItem from '@mui/material/MenuItem';
import Select, { SelectChangeEvent } from '@mui/material/Select';
import TextField from '@mui/material/TextField';
import Stack from '@mui/material/Stack';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import { useNavigate } from 'react-router';
import dayjs, { Dayjs } from 'dayjs';
import { DiaSemana } from '../modelos/usuarios';
import { FULL_HEIGHT_WIDTH } from '../context/DashboardSidebarContext';
import { useIsMounted } from '../hooks/useComponentMounted';
import { DatePicker } from '@mui/x-date-pickers/DatePicker';
import LocalizationProviderES from '../theme/location';
import { validarFechaHora } from '../error';

export interface HorarioFormValues {
  dia: DiaSemana;
  horas: number;
  cortesia: number;
  caducidadFechaIni: Dayjs | null;
  caducidadFechaFin: Dayjs | null;
}

export interface HorarioFormState {
  values: HorarioFormValues;
  errors: Partial<Record<keyof HorarioFormValues, string>>;
}

export type FormFieldValue = string | number | Dayjs | null;

export interface HorarioFormProps {
  formState: HorarioFormState;
  onFieldChange: (
    name: keyof HorarioFormValues,
    value: FormFieldValue,
  ) => void;
  onSubmit: (formValues: HorarioFormValues) => Promise<void>;
  onReset?: () => void;
  submitButtonLabel: string;
  backButtonPath?: string;
  onBack?: () => void;
}

export type ValidationResult = {
  issues: { message: string; path: (keyof HorarioFormValues)[] }[]
};

export function validaHorario(values: HorarioFormValues): ValidationResult {
  let issues: ValidationResult['issues'] = [];

  if (!values.dia) {
    issues.push({ message: 'El día es requerido', path: ['dia'] });
  }

  if (!values.horas) {
    issues.push({ message: 'Las horas son requeridas', path: ['horas'] });
  } else if (values.horas < 1 || values.horas > 24) {
    issues.push({ message: 'Las horas deben estar entre 1 y 24', path: ['horas'] });
  }

  if (values.cortesia < 0 || values.cortesia > 120) {
    issues.push({ message: 'Los minutos de cortesía deben estar entre 0 y 120', path: ['cortesia'] });
  }

  if (values.caducidadFechaIni) {
    if (!validarFechaHora(values.caducidadFechaIni)) {
      issues.push({ message: 'Fecha inválida', path: ['caducidadFechaIni'] });
    } else if (values.caducidadFechaIni.isBefore(dayjs(), 'day')) {
      issues.push(
        {
          message: 'La fecha inicio de caducidad debe ser igual o mayor al día de hoy',
          path: ['caducidadFechaIni']
        });
    }
  }

  if (values.caducidadFechaFin) {
    if (!validarFechaHora(values.caducidadFechaFin)) {
      issues.push({ message: 'Fecha inválida', path: ['caducidadFechaFin'] });
    } else if (values.caducidadFechaIni && values.caducidadFechaFin.isBefore(values.caducidadFechaIni, 'day')) {
      issues.push(
        {
          message: 'La fecha fin de caducidad debe ser igual o mayor a la fecha inicio',
          path: ['caducidadFechaFin']
        });

    }
  }

  if (values.caducidadFechaIni && !values.caducidadFechaFin) {
    issues.push(
      {
        message: 'La fecha fin de caducidad es requerida si se especifica la fecha inicio',
        path: ['caducidadFechaFin']
      });
  }

  if (values.caducidadFechaFin && !values.caducidadFechaIni) {
    issues.push(
      {
        message: 'La fecha inicio de caducidad es requerida si se especifica la fecha fin',
        path: ['caducidadFechaIni']
      });
  }

  return { issues };
}

export default function HorarioForm(props: HorarioFormProps) {
  const {
    formState,
    onFieldChange,
    onSubmit,
    onReset,
    submitButtonLabel,
    backButtonPath,
    onBack,
  } = props;

  const navigate = useNavigate();
  const isMounted = useIsMounted();

  const formValues = formState.values;
  const formErrors = formState.errors;

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
    (event: SelectChangeEvent) => {
      onFieldChange(
        event.target.name as keyof HorarioFormValues,
        event.target.value,
      );
    },
    [onFieldChange],
  );

  const handleNumberFieldChange = React.useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      onFieldChange(
        event.target.name as keyof HorarioFormValues,
        Number(event.target.value),
      );
    },
    [onFieldChange],
  );

  const handleDateFieldChange = React.useCallback(
    (name: keyof HorarioFormValues) => (value: Dayjs | null) => {
      onFieldChange(name, value);
    },
    [onFieldChange],
  );

  const handleBack = React.useCallback(() => {
    if (onBack) {
      onBack();
    } else {
      navigate(backButtonPath ?? '/horarios');
    }
  }, [navigate, backButtonPath, onBack]);

  const handleReset = React.useCallback(() => {
    if (onReset) {
      onReset();
    }
  }, [onReset]);

  // Obtener lista de días para el select
  const diasOptions = React.useMemo(() => {
    return Object.entries(DiaSemana).map(([key, value]) => ({
      label: key,
      value: value
    }));
  }, []);

  return (
    <Box
      component="form"
      onSubmit={handleSubmit}
      noValidate
      autoComplete="off"
      onReset={handleReset}
      sx={FULL_HEIGHT_WIDTH}
    >
      <LocalizationProviderES>
        <Grid container spacing={2} sx={{ mt: 2, mb: 2, width: '100%' }}>
          <Grid size={{ xs: 12 }} sx={{ display: 'flex' }}>
            <FormControl error={!!formErrors.dia} fullWidth>
              <InputLabel>Día</InputLabel>
              <Select
                name="dia"
                value={formValues.dia}
                label="Día"
                onChange={handleSelectFieldChange}
              >
                {diasOptions.map((option) => (
                  <MenuItem key={option.value} value={option.value}>
                    {option.label}
                  </MenuItem>
                ))}
              </Select>
              <FormHelperText>{formErrors.dia ?? ' '}</FormHelperText>
            </FormControl>
          </Grid>

          <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex' }}>
            <TextField
              name="horas"
              label="Horas a trabajar"
              type="number"
              value={formValues.horas}
              onChange={handleNumberFieldChange}
              error={!!formErrors.horas}
              helperText={formErrors.horas ?? ' '}
              fullWidth
              slotProps={{
                htmlInput: { min: 1, max: 24, step: 1 }
              }}
            />
          </Grid>

          <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex' }}>
            <TextField
              name="cortesia"
              label="Minutos de cortesía"
              type="number"
              value={formValues.cortesia}
              onChange={handleNumberFieldChange}
              error={!!formErrors.cortesia}
              helperText={formErrors.cortesia ?? ' '}
              fullWidth
              slotProps={{
                htmlInput: { min: 0, max: 120, step: 1 }
              }}
            />
          </Grid>

          <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex' }}>
            <DatePicker
              label="Fecha inicio (Caducidad)"
              value={formValues.caducidadFechaIni}
              onChange={handleDateFieldChange('caducidadFechaIni')}
              slotProps={{
                textField: {
                  fullWidth: true,
                  error: !!formErrors.caducidadFechaIni,
                  helperText: formErrors.caducidadFechaIni ?? ' '
                }
              }}
            />
          </Grid>

          <Grid size={{ xs: 12, sm: 6 }} sx={{ display: 'flex' }}>
            <DatePicker
              label="Fecha fin (Caducidad)"
              value={formValues.caducidadFechaFin}
              onChange={handleDateFieldChange('caducidadFechaFin')}
              slotProps={{
                textField: {
                  fullWidth: true,
                  error: !!formErrors.caducidadFechaFin,
                  helperText: formErrors.caducidadFechaFin ?? ' '
                }
              }}
            />
          </Grid>
        </Grid>
      </LocalizationProviderES>

      <Stack direction="row" spacing={2} justifyContent="space-between">
        <Button
          variant="contained"
          startIcon={<ArrowBackIcon />}
          onClick={handleBack}
        >
          VOLVER
        </Button>
        <Button type="submit" variant="contained" size="large" loading={isSubmitting}>
          {submitButtonLabel}
        </Button>
      </Stack>
    </Box>
  );
}