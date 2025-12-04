import { useState, useEffect, useCallback } from 'react';
import InputLabel from '@mui/material/InputLabel';
import Select, { SelectChangeEvent } from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import { DescriptorUsuario, RolID } from '../modelos/usuarios';
import { api } from '../api/fabrica';
import { NetErrorControlado } from '../net/interceptor';
import useNotifications from '../hooks/useNotifications/useNotifications';
import { logError } from '../error';
import { useIsMounted } from '../hooks/useComponentMounted';

interface SelectorEmpleadoProps {
  onChange: (empleado: DescriptorUsuario) => void;
  onLoadingChange?: (isLoading: boolean) => void;
  usuarioPorDefecto?: number;
  disabled?: boolean;
  label?: string;
  fullWidth?: boolean;
}

export default function SelectorEmpleado({
  onChange,
  disabled = false,
  label = 'Empleado',
  fullWidth = true,
  onLoadingChange,
  usuarioPorDefecto: seleccionUsuario
}: SelectorEmpleadoProps) {
  const isMounted = useIsMounted();
  const notifica = useNotifications();

  const [empleados, setEmpleados] = useState<DescriptorUsuario[]>([]);
  const [empleado, setEmpleado] =
    useState<DescriptorUsuario | undefined>(undefined);
  const [isLoading, setIsLoading] = useState<boolean>(true);

  const cargarEmpleados = useCallback(async () => {
    setIsLoading(true);
    if (onLoadingChange) onLoadingChange(true);

    let empls: DescriptorUsuario[] = []

    try {
      empls = await api().usuarios.usuariosPorRol(RolID.Empleado);

      if (empls.length > 0) {
        let empleadoASeleccionar: DescriptorUsuario | undefined;

        // Si se especificó seleccionUsuario, buscar ese usuario
        if (seleccionUsuario !== undefined) {
          empleadoASeleccionar = empls.find(e => e.id === seleccionUsuario);
        }

        // Si no se encontró o no se especificó, seleccionar el primero
        if (!empleadoASeleccionar) {
          empleadoASeleccionar = empls[0];
        }

        if (isMounted.current) {
          setEmpleado(empleadoASeleccionar);
          onChange(empleadoASeleccionar);
        };
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
    }

    if (isMounted.current) {
      setEmpleados(empls);
      setIsLoading(false);
      if (onLoadingChange) onLoadingChange(false);
    };
  }, [onLoadingChange, onChange, notifica]);

  useEffect(() => {
    cargarEmpleados();
  }, [cargarEmpleados]);

  const handleChange = useCallback(
    (event: SelectChangeEvent<string>) => {
      const id = Number(event.target.value);
      const empleadoSeleccionado = empleados.find(u => u.id === id);
      setEmpleado(empleadoSeleccionado)
      if (empleadoSeleccionado) {
        onChange(empleadoSeleccionado);
      }
    },
    [empleados, onChange]
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