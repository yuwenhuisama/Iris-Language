// Iris Pointer Extension.cpp : 定义 DLL 应用程序的导出函数。
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

