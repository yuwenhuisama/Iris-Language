// ���� ifdef ���Ǵ���ʹ�� DLL �������򵥵�
// ��ı�׼�������� DLL �е������ļ��������������϶���� IRISPOINTEREXTENSION_EXPORTS
// ���ű���ġ���ʹ�ô� DLL ��
// �κ�������Ŀ�ϲ�Ӧ����˷��š�������Դ�ļ��а������ļ����κ�������Ŀ���Ὣ
// IRISPOINTEREXTENSION_API ������Ϊ�Ǵ� DLL ����ģ����� DLL ���ô˺궨���
// ������Ϊ�Ǳ������ġ�
#ifdef IRISPOINTEREXTENSION_EXPORTS
#define IRISPOINTEREXTENSION_API __declspec(dllexport)
#else
#define IRISPOINTEREXTENSION_API __declspec(dllimport)
#endif

#ifdef __cplusplus
extern "C" {
#endif
	IRISPOINTEREXTENSION_API bool IR_Initialize();
	IRISPOINTEREXTENSION_API bool IR_Release();
#ifdef __cplusplus
}
#endif