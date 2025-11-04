import { useState, useEffect, useCallback } from 'react';
import InputLabel from '@mui/material/InputLabel';
import Select, { SelectChangeEvent } from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import { DescriptorUsuario, RolID } from '../modelos/usuarios';
import { api } from '../api/fabrica';
import { NetErrorControlado } from '../net/interceptor';
import useNotifications from '../hooks/useNotifications/useNotifications';
import { logError } from '../error';

interface SelectorEmpleadoProps {
  onChange: (empleado: DescriptorUsuario | undefined) => void;
  onLoadingChange?: (isLoading: boolean) => void;
  disabled?: boolean;
  label?: string;
  fullWidth?: boolean;
}

export default function SelectorEmpleado({
  onChange,
  disabled = false,
  label = 'Empleado',
  fullWidth = true,
  onLoadingChange
}: SelectorEmpleadoProps) {
  const [empleados, setEmpleados] = useState<DescriptorUsuario[]>([]);
  const [empleado, setEmpleado] =
    useState<DescriptorUsuario | undefined>(undefined);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const notifica = useNotifications();

  const cargarEmpleados = useCallback(async () => {
    setIsLoading(true);
    if (onLoadingChange) onLoadingChange(true);

    try {
      const empls = await api().usuarios.usuariosPorRol(RolID.Empleado);
      setEmpleados(empls);

      if (empls.length > 0) {
        setEmpleado(empls[0])
        onChange(empls[0]);
      }
    } catch (error) {
      if (!(error instanceof NetErrorControlado)) {
        logError('selector-empleado.cargar', error);
        notifica.show(
          'Error inesperado al cargar los empleados',
          {
            severity: 'error',
            autoHideDuration: 5000,
          }
        );
      }
      setEmpleados([]);
    } finally {
      setIsLoading(false);
      if (onLoadingChange) onLoadingChange(false);
    }
  }, [onLoadingChange, onChange]);

  useEffect(() => {
    cargarEmpleados();
  }, [cargarEmpleados]);

  const handleChange = useCallback(
    (event: SelectChangeEvent<string>) => {
      const id = Number(event.target.value);
      const empleadoSeleccionado = empleados.find(u => u.id === id);
      setEmpleado(empleadoSeleccionado)
      onChange(empleadoSeleccionado);
    },
    [empleados]
  );

  return (
    <>
      <InputLabel>{label}</InputLabel>
      <Select
        name="empleado"
        value={empleado?.id?.toString() ?? ''}
        label={label}
        onChange={handleChange}
        disabled={disabled || isLoading}
        fullWidth={fullWidth}
      >
        {empleados.map((empleado) => (
          <MenuItem key={empleado.id} value={empleado.id.toString()}>
            {empleado.nombreCompleto()}
          </MenuItem>
        ))}
      </Select>
    </>
  );
}