// ���� ifdef ���Ǵ���ʹ�� DLL �������򵥵�
// ��ı�׼�������� DLL �е������ļ��������������϶���� IRISLANGLIBRARY_EXPORTS
// ���ű���ġ���ʹ�ô� DLL ��
// �κ�������Ŀ�ϲ�Ӧ����˷��š�������Դ�ļ��а������ļ����κ�������Ŀ���Ὣ
// IRISLANGLIBRARY_API ������Ϊ�Ǵ� DLL ����ģ����� DLL ���ô˺궨���
// ������Ϊ�Ǳ������ġ�

#include "IrisCompileConfigure.h"

#if IR_USE_C_EXPORT
#include "IrisExportAPIs\IrisExportForC.h"
#else
#include "IrisExportAPIs\IrisExportForCpp.h"
#endif