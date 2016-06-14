#include "IrisInterpreter\IrisNativeClasses\IrisIrregular.h"



IrisIrregular::IrisIrregular()
{
}


IrisIrregular::~IrisIrregular()
{
}

IrisValue IrisIrregular::InitializeFunction(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment)
{
	auto pIrregular = IrisDevUtil::GetNativePointer<IrisIrregularTag*>(ivObj);

	auto& ivLineNumber = ivsValues->GetValue(0);
	auto& ivFileName = ivsValues->GetValue(1);
	auto& ivMsg = ivsValues->GetValue(2);

	if (!IrisDevUtil::CheckClassIsInteger(ivLineNumber)) {
		IrisDevUtil::GroanIrregularWithString("Invaild parameter 1 which must be an Integer.");
		return ivObj;
	}
	if (!IrisDevUtil::CheckClassIsString(ivFileName) && !IrisDevUtil::CheckClassIsUniqueString(ivFileName)) {
		IrisDevUtil::GroanIrregularWithString("Invaild parameter 2 which must be a String or an UniqueString.");
		return ivObj;
	}
	if (!IrisDevUtil::CheckClassIsString(ivMsg) && !IrisDevUtil::CheckClassIsUniqueString(ivMsg)) {
		IrisDevUtil::GroanIrregularWithString("Invaild parameter 2 which must be an Integer or an UniqueString.");
		return ivObj;
	}
	auto nLineNumber = IrisDevUtil::GetInt(ivLineNumber);
	auto strFileName = IrisDevUtil::GetString(ivFileName);
	auto strMsg = IrisDevUtil::GetString(ivMsg);

	IrisDevUtil::SetObjectInstanceVariable(ivObj, "@line_number", ivLineNumber);
	IrisDevUtil::SetObjectInstanceVariable(ivObj, "@file_name", ivFileName);
	IrisDevUtil::SetObjectInstanceVariable(ivObj, "@message", ivMsg);
	
	pIrregular->Initialize(nLineNumber, strFileName, strMsg);

	return ivObj;
}

IrisValue IrisIrregular::SetLineNumber(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment)
{
	auto& ivLineNumber = ivsValues->GetValue(0);
	if (!IrisDevUtil::CheckClassIsInteger(ivLineNumber)) {
		IrisDevUtil::GroanIrregularWithString("Invaild parameter 1 which must be an Integer.");
		return IrisDevUtil::Nil();
	}
	IrisDevUtil::SetObjectInstanceVariable(ivObj, "@line_number", ivLineNumber);
	return IrisDevUtil::Nil();
}

IrisValue IrisIrregular::SetFileName(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment)
{
	auto& ivFileName = ivsValues->GetValue(0);
	if (!IrisDevUtil::CheckClassIsString(ivFileName) && !IrisDevUtil::CheckClassIsUniqueString(ivFileName)) {
		IrisDevUtil::GroanIrregularWithString("Invaild parameter 1 which must be a String or an Integer.");
		return IrisDevUtil::Nil();
	}
	IrisDevUtil::SetObjectInstanceVariable(ivObj, "@file_name", ivFileName);
	return IrisDevUtil::Nil();
}

IrisValue IrisIrregular::SetMessage(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment)
{
	auto& ivMsg = ivsValues->GetValue(0);
	if (!IrisDevUtil::CheckClassIsString(ivMsg) && !IrisDevUtil::CheckClassIsUniqueString(ivMsg)) {
		IrisDevUtil::GroanIrregularWithString("Invaild parameter 1 which must be a String or an Integer.");
		return IrisDevUtil::Nil();
	}
	IrisDevUtil::SetObjectInstanceVariable(ivObj, "@message", ivMsg);
	return IrisDevUtil::Nil();
}

IrisValue IrisIrregular::GetLineNumber(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment)
{
	return IrisDevUtil::GetObjectInstanceVariable(ivObj, "@line_number");
}

IrisValue IrisIrregular::GetFileName(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment)
{
	return IrisDevUtil::GetObjectInstanceVariable(ivObj, "@file_name");
}

IrisValue IrisIrregular::GetMessage(IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment)
{
	return IrisDevUtil::GetObjectInstanceVariable(ivObj, "@message");
}
 