// Iris Pointer Extension.cpp : ���� DLL Ӧ�ó���ĵ���������
//

#include "stdafx.h"
#include "Iris Pointer Extension.h"
#include "IrisPointer.h"

IRISPOINTEREXTENSION_API bool IR_Initialize()
{
	IrisDev::RegistClass("Pointer", new IrisPointer());

	return true;
}

IRISPOINTEREXTENSION_API bool IR_Release()
{
	return true;
}

