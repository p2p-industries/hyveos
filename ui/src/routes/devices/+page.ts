import { DeviceStatusTag, type DeviceType } from '$lib/types';
import type { PageLoad } from './$types';

export const load: PageLoad = async () => {
  const devices: DeviceType[] = [
    {
      id: '1',
      name: 'Device 1',
      status: {
        tag: DeviceStatusTag.Ok
      },
      dockerImages: [
        { name: 'nginx:latest', status: 'running' },
        { name: 'redis:alpine', status: 'stopped' }
      ]
    },
    {
      id: '2',
      name: 'Device 2',
      status: {
        tag: DeviceStatusTag.Error,
        error: 'Error message'
      },
      dockerImages: [
        { name: 'postgres:12.0', status: 'running' },
        { name: 'httpd:2.4', status: 'running' }
      ]
    }
    // Add more devices as needed
  ];
  return {
    devices
  };
};
