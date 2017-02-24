#include "IrisInterpreter/IrisNativeClasses/IrisUniqueString.h"


unordered_map<size_t, IrisValue> IrisUniqueString::s_mpUniqueStringMap;

const IrisValue & IrisUniqueString::GetUniqueString(size_t nIndex, bool & bResult) {
	unordered_map<size_t, IrisValue>::iterator iStr;
	if ((iStr = s_mpUniqueStringMap.find(nIndex)) != s_mpUniqueStringMap.end()) {
		bResult = true;
		return iStr->second;
	}
	bResult = false;
	return IrisDevUtil::Nil();
}

void IrisUniqueString::AddUniqueString(size_t nIndex, const IrisValue & ivUniqueString) {
	s_mpUniqueStringMap[nIndex] = ivUniqueString;
}

IrisValue IrisUniqueString::InitializeFunction(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	return ivObj;
}

IrisValue IrisUniqueString::ToString(const IrisValue & ivObj, IIrisValues * ivsValues, IIrisValues * ivsVariableValues, IIrisContextEnvironment * pContextEnvironment) {
	auto pUniqueString = IrisDevUtil::GetNativePointer<IrisUniqueStringTag*>(ivObj);
	return IrisDevUtil::CreateString(pUniqueString->GetString().c_str());
}

IrisUniqueString::IrisUniqueString()
{
}

IrisUniqueString::~IrisUniqueString()
{
}
